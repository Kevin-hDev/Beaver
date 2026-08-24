if ($null -eq ("Beaver.Release.NativeIcon" -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

namespace Beaver.Release {
    public static class NativeIcon {
        [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        public static extern uint PrivateExtractIcons(
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
        public static extern bool DestroyIcon(IntPtr iconHandle);
    }
}
'@
}

function Get-FixedSizeNativeIcon([string]$Path, [int]$Size) {
    if (
        [string]::IsNullOrWhiteSpace($Path) -or
        $Size -le 0 -or
        $Size -gt 256 -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf)
    ) {
        throw "Invalid icon extraction input."
    }

    # One native path for EXE and ICO avoids DPI and .NET decoder differences.
    Add-Type -AssemblyName System.Drawing
    $handles = New-Object IntPtr[] 1
    $iconIds = New-Object uint32[] 1
    $count = [Beaver.Release.NativeIcon]::PrivateExtractIcons(
        [IO.Path]::GetFullPath($Path),
        0,
        $Size,
        $Size,
        $handles,
        $iconIds,
        1,
        0
    )
    if ($handles[0] -eq [IntPtr]::Zero) {
        throw "Native icon extraction failed."
    }

    $clonedIcon = $null
    try {
        if ($count -ne 1) {
            throw "Native icon extraction failed."
        }
        $borrowedIcon = [Drawing.Icon]::FromHandle($handles[0])
        try {
            $clonedIcon = [Drawing.Icon]$borrowedIcon.Clone()
        } finally {
            $borrowedIcon.Dispose()
        }
    } finally {
        $destroyed = [Beaver.Release.NativeIcon]::DestroyIcon($handles[0])
    }
    if (-not $destroyed) {
        if ($null -ne $clonedIcon) {
            $clonedIcon.Dispose()
        }
        throw "Native icon cleanup failed."
    }
    return $clonedIcon
}
