. (Join-Path $PSScriptRoot "windows-package-file.ps1")

Add-Type -AssemblyName System.Drawing

$nativeIconInteropTemplate = @'
using System;
using System.Runtime.InteropServices;

namespace Beaver.Release {
    // The content hash in the class name reloads edited interop code in long-lived shells.
    public static class __CLASS__ {
        [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern uint PrivateExtractIcons(
            string fileName,
            int iconIndex,
            int width,
            int height,
            IntPtr[] iconHandles,
            uint[] iconIds,
            uint iconCount,
            uint flags
        );

        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool DestroyIcon(IntPtr iconHandle);

        public static IntPtr Extract(string fileName, int size) {
            var handles = new IntPtr[1];
            var iconIds = new uint[1];
            var count = PrivateExtractIcons(fileName, 0, size, size, handles, iconIds, 1, 0);
            if (count == 1 && handles[0] != IntPtr.Zero) {
                return handles[0];
            }
            if (handles[0] != IntPtr.Zero && !DestroyIcon(handles[0])) {
                return new IntPtr(-1);
            }
            return IntPtr.Zero;
        }

        public static bool Release(IntPtr iconHandle) {
            return DestroyIcon(iconHandle);
        }
    }
}
'@

$nativeIconSha256 = [Security.Cryptography.SHA256]::Create()
try {
    $nativeIconSourceBytes = [Text.Encoding]::UTF8.GetBytes($nativeIconInteropTemplate)
    $nativeIconHash = -join @(
        $nativeIconSha256.ComputeHash($nativeIconSourceBytes) |
            ForEach-Object { $_.ToString("x2") }
    )
} finally {
    $nativeIconSha256.Dispose()
}
$nativeIconClassName = "NativeIcon_$nativeIconHash"
$nativeIconTypeName = "Beaver.Release.$nativeIconClassName"
$script:NativeIconInteropType = $nativeIconTypeName -as [type]
if ($null -eq $script:NativeIconInteropType) {
    $nativeIconInteropSource = $nativeIconInteropTemplate.Replace("__CLASS__", $nativeIconClassName)
    $compiledTypes = @(Add-Type -TypeDefinition $nativeIconInteropSource -PassThru)
    $script:NativeIconInteropType = @(
        $compiledTypes | Where-Object { $_.FullName -ceq $nativeIconTypeName }
    )[0]
}
if ($null -eq $script:NativeIconInteropType) {
    throw (New-Object Runtime.InteropServices.ExternalException("Native icon runtime failed."))
}

function Get-FixedSizeNativeIcon([IO.FileInfo]$File, [int]$Size) {
    if ($null -eq $File -or $Size -le 0 -or $Size -gt 256) {
        throw (New-Object ArgumentException("Invalid icon extraction input."))
    }

    $arguments = New-Object object[] 2
    $arguments[0] = $File.FullName
    $arguments[1] = $Size
    try {
        $handle = [IntPtr]$script:NativeIconInteropType.GetMethod("Extract").Invoke(
            $null,
            $arguments
        )
    } catch {
        throw (New-Object Runtime.InteropServices.ExternalException("Native icon runtime failed."))
    }
    if ($handle -eq [IntPtr](-1)) {
        throw (New-Object Runtime.InteropServices.ExternalException("Native icon runtime failed."))
    }
    if ($handle -eq [IntPtr]::Zero) {
        throw (New-Object IO.InvalidDataException("Native icon extraction failed."))
    }

    $clonedIcon = $null
    try {
        try {
            $borrowedIcon = [Drawing.Icon]::FromHandle($handle)
            try {
                $clonedIcon = [Drawing.Icon]$borrowedIcon.Clone()
            } finally {
                $borrowedIcon.Dispose()
            }
        } catch {
            throw (New-Object Runtime.InteropServices.ExternalException("Native icon runtime failed."))
        }
    } finally {
        try {
            $destroyed = [bool]$script:NativeIconInteropType.GetMethod("Release").Invoke(
                $null,
                [object[]]@($handle)
            )
        } catch {
            $destroyed = $false
        }
        if (-not $destroyed) {
            if ($null -ne $clonedIcon) {
                $clonedIcon.Dispose()
            }
            throw (New-Object Runtime.InteropServices.ExternalException("Native icon runtime failed."))
        }
    }
    return $clonedIcon
}

function Get-NativeIconResourceFailure([string]$Path, [long]$MaxBytes) {
    $icon = $null
    try {
        try {
            $file = Get-BoundedPackageFile $Path $MaxBytes
        } catch {
            return "input"
        }
        try {
            $icon = Get-FixedSizeNativeIcon $file 32
        } catch [Runtime.InteropServices.ExternalException] {
            return "runtime"
        } catch {
            return "extract"
        }
        return $null
    } catch {
        return "runtime"
    } finally {
        if ($null -ne $icon) {
            $icon.Dispose()
        }
    }
}
