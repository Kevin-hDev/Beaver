[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$SourceExecutable,
    [Parameter(Mandatory = $true)][string]$DestinationExecutable,
    [Parameter(Mandatory = $true)][string]$ExpectedProductName,
    [Parameter(Mandatory = $true)][string]$ExpectedVersion
)

$ErrorActionPreference = "Stop"
$MaxExecutableBytes = 536870912

$resourceApi = @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class BeaverResourceBranding {
    const uint LoadAsData = 0x00000002;
    const uint LoadAsImageResource = 0x00000020;
    const int MaxResources = 256;
    const int MaxResourceBytes = 16 * 1024 * 1024;
    const int MaxTotalBytes = 32 * 1024 * 1024;
    static readonly ushort[] Types = { 3, 14, 16 };

    sealed class Resource {
        internal ushort Type;
        internal ushort Name;
        internal ushort Language;
        internal byte[] Data;
    }

    sealed class ReadState {
        internal readonly List<Resource> Resources = new List<Resource>();
        internal Exception Error;
        internal int TotalBytes;
        internal readonly bool IncludeData;
        internal ReadState(bool includeData) { IncludeData = includeData; }
    }

    delegate bool EnumName(IntPtr module, IntPtr type, IntPtr name, IntPtr value);
    delegate bool EnumLanguage(IntPtr module, IntPtr type, IntPtr name, ushort language, IntPtr value);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern IntPtr LoadLibraryExW(string path, IntPtr file, uint flags);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool FreeLibrary(IntPtr module);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool EnumResourceNamesW(IntPtr module, IntPtr type, EnumName callback, IntPtr value);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool EnumResourceLanguagesW(IntPtr module, IntPtr type, IntPtr name, EnumLanguage callback, IntPtr value);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern IntPtr FindResourceExW(IntPtr module, IntPtr type, IntPtr name, ushort language);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern uint SizeofResource(IntPtr module, IntPtr resource);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr LoadResource(IntPtr module, IntPtr resource);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr LockResource(IntPtr resource);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern IntPtr BeginUpdateResourceW(string path, bool deleteExisting);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool UpdateResourceW(IntPtr update, IntPtr type, IntPtr name, ushort language, byte[] data, uint size);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool EndUpdateResourceW(IntPtr update, bool discard);

    static bool IsIntegerResource(IntPtr value) {
        return ((ulong)value.ToInt64() >> 16) == 0;
    }

    static List<Resource> Read(string path, bool includeData) {
        IntPtr module = LoadLibraryExW(path, IntPtr.Zero, LoadAsData | LoadAsImageResource);
        if (module == IntPtr.Zero) throw new InvalidOperationException();
        try {
            var state = new ReadState(includeData);
            foreach (ushort resourceType in Types) {
                IntPtr type = new IntPtr(resourceType);
                EnumName names = (loaded, currentType, name, ignored) => {
                    if (!IsIntegerResource(name)) {
                        state.Error = new InvalidOperationException();
                        return false;
                    }
                    EnumLanguage languages = (languageModule, languageType, languageName, language, unused) => {
                        try {
                            if (state.Resources.Count >= MaxResources) throw new InvalidOperationException();
                            byte[] bytes = null;
                            if (state.IncludeData) {
                                IntPtr found = FindResourceExW(languageModule, languageType, languageName, language);
                                uint size = found == IntPtr.Zero ? 0 : SizeofResource(languageModule, found);
                                if (size == 0 || size > MaxResourceBytes) throw new InvalidOperationException();
                                state.TotalBytes = checked(state.TotalBytes + (int)size);
                                if (state.TotalBytes > MaxTotalBytes) throw new InvalidOperationException();
                                IntPtr loadedResource = LoadResource(languageModule, found);
                                IntPtr data = loadedResource == IntPtr.Zero ? IntPtr.Zero : LockResource(loadedResource);
                                if (data == IntPtr.Zero) throw new InvalidOperationException();
                                bytes = new byte[size];
                                Marshal.Copy(data, bytes, 0, (int)size);
                            }
                            state.Resources.Add(new Resource {
                                Type = resourceType,
                                Name = (ushort)languageName.ToInt64(),
                                Language = language,
                                Data = bytes
                            });
                            return true;
                        } catch (Exception error) {
                            state.Error = error;
                            return false;
                        }
                    };
                    EnumResourceLanguagesW(loaded, currentType, name, languages, IntPtr.Zero);
                    return state.Error == null;
                };
                EnumResourceNamesW(module, type, names, IntPtr.Zero);
                if (state.Error != null) throw state.Error;
            }
            return state.Resources;
        } finally {
            FreeLibrary(module);
        }
    }

    static void Verify(List<Resource> expected, List<Resource> actual) {
        if (actual.Count != expected.Count) throw new InvalidOperationException();
        var indexed = new Dictionary<string, byte[]>();
        foreach (Resource item in actual) {
            string key = item.Type + ":" + item.Name + ":" + item.Language;
            if (item.Data == null || indexed.ContainsKey(key)) throw new InvalidOperationException();
            indexed.Add(key, item.Data);
        }
        foreach (Resource item in expected) {
            string key = item.Type + ":" + item.Name + ":" + item.Language;
            byte[] bytes;
            if (!indexed.TryGetValue(key, out bytes) || bytes.Length != item.Data.Length)
                throw new InvalidOperationException();
            for (int index = 0; index < bytes.Length; index++) {
                if (bytes[index] != item.Data[index]) throw new InvalidOperationException();
            }
        }
    }

    public static void Copy(string source, string destination) {
        List<Resource> sourceResources = Read(source, true);
        List<Resource> destinationResources = Read(destination, false);
        foreach (ushort type in Types) {
            if (!sourceResources.Exists(item => item.Type == type)) throw new InvalidOperationException();
        }

        IntPtr update = BeginUpdateResourceW(destination, false);
        if (update == IntPtr.Zero) throw new InvalidOperationException();
        try {
            foreach (Resource item in destinationResources) {
                if (!UpdateResourceW(update, new IntPtr(item.Type), new IntPtr(item.Name), item.Language, null, 0))
                    throw new InvalidOperationException();
            }
            foreach (Resource item in sourceResources) {
                if (!UpdateResourceW(update, new IntPtr(item.Type), new IntPtr(item.Name), item.Language, item.Data, (uint)item.Data.Length))
                    throw new InvalidOperationException();
            }
            if (!EndUpdateResourceW(update, false)) throw new InvalidOperationException();
            update = IntPtr.Zero;
        } finally {
            if (update != IntPtr.Zero) EndUpdateResourceW(update, true);
        }
        Verify(sourceResources, Read(destination, true));
    }
}
'@

try {
    if ($ExpectedProductName -notmatch "^[A-Za-z0-9 ._-]{1,64}$" -or $ExpectedVersion -notmatch "^[0-9]+\.[0-9]+\.[0-9]+$") {
        throw "invalid"
    }
    foreach ($providedPath in @($SourceExecutable, $DestinationExecutable)) {
        if (
            [string]::IsNullOrWhiteSpace($providedPath) -or
            -not [IO.Path]::IsPathRooted($providedPath) -or
            $providedPath -match "(^|[\\/])\.\.([\\/]|$)"
        ) {
            throw "invalid"
        }
    }
    $source = [IO.Path]::GetFullPath($SourceExecutable)
    $destination = [IO.Path]::GetFullPath($DestinationExecutable)
    if ($source.Equals($destination, [StringComparison]::OrdinalIgnoreCase)) {
        throw "invalid"
    }
    foreach ($path in @($source, $destination)) {
        $item = Get-Item -LiteralPath $path -Force
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if ($item.PSIsContainer -or $isLink -or $item.Length -le 0 -or $item.Length -gt $MaxExecutableBytes) {
            throw "invalid"
        }
    }
    if (-not ("BeaverResourceBranding" -as [type])) {
        Add-Type -TypeDefinition $resourceApi -Language CSharp
    }
    [BeaverResourceBranding]::Copy($source, $destination)
    $result = Get-Item -LiteralPath $destination -Force
    if ($result.VersionInfo.ProductName -cne $ExpectedProductName -or $result.VersionInfo.FileVersion -cne $ExpectedVersion) {
        throw "invalid"
    }
} catch {
    throw "Windows executable branding failed."
}
