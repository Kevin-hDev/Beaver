Var BeaverLegacyMigration
Var BeaverLegacyDir

!define BEAVER_OLD_UNINSTALL "Software\Microsoft\Windows\CurrentVersion\Uninstall\CL-GO"
!define BEAVER_NEW_UNINSTALL "Software\Microsoft\Windows\CurrentVersion\Uninstall\Beaver"
!define BEAVER_OLD_PRODUCT "Software\clgo\CL-GO"
!define BEAVER_NEW_PRODUCT "Software\clgo\Beaver"
!define BEAVER_MAIN_BINARY "cl-go-dash.exe"

!macro BeaverDeleteLegacyShortcut ShortcutPath
  !insertmacro IsShortcutTarget "${ShortcutPath}" "$INSTDIR\${BEAVER_MAIN_BINARY}"
  Pop $R8
  ${If} $R8 = 1
    !insertmacro UnpinShortcut "${ShortcutPath}"
    Delete "${ShortcutPath}"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREINSTALL
  StrCpy $BeaverLegacyMigration 0
  StrCpy $BeaverLegacyDir ""

  ReadRegStr $R8 SHCTX "${BEAVER_OLD_UNINSTALL}" "UninstallString"
  ${If} $R8 == ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}

  ReadRegStr $R8 SHCTX "${BEAVER_OLD_PRODUCT}" ""
  ${If} $R8 == ""
    ReadRegStr $R8 SHCTX "${BEAVER_OLD_UNINSTALL}" "InstallLocation"
  ${EndIf}

  StrCpy $R9 $R8 1
  ${If} $R9 == '"'
    StrCpy $R9 $R8 1 -1
    ${If} $R9 != '"'
      Goto beaver_legacy_preinstall_done
    ${EndIf}
    StrCpy $R8 $R8 "" 1
    StrCpy $R8 $R8 -1
  ${EndIf}

  StrLen $R9 $R8
  ${If} $R9 < 4
  ${OrIf} $R9 > 1024
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  StrCpy $R9 $R8 1 1
  ${If} $R9 != ":"
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  StrCpy $R9 $R8 1 2
  ${If} $R9 != "\"
    Goto beaver_legacy_preinstall_done
  ${EndIf}

  ${StrLoc} $R9 $R8 ".." ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 '"' ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 "/" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 "*" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 "?" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 "<" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 ">" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 "|" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  ${StrLoc} $R9 $R8 "\.\" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}
  StrCpy $R7 $R8 "" 2
  ${StrLoc} $R9 $R7 ":" ">"
  ${If} $R9 != ""
    Goto beaver_legacy_preinstall_done
  ${EndIf}

  IfFileExists "$R8\${BEAVER_MAIN_BINARY}" 0 beaver_legacy_preinstall_done
  ${If} $INSTDIR == "$LOCALAPPDATA\${PRODUCTNAME}"
    StrCpy $INSTDIR $R8
  ${ElseIf} $INSTDIR != $R8
    Goto beaver_legacy_preinstall_done
  ${EndIf}

  StrCpy $BeaverLegacyDir $R8
  StrCpy $BeaverLegacyMigration 1
  SetOutPath $INSTDIR

  beaver_legacy_preinstall_done:
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${If} $BeaverLegacyMigration != 1
    Goto beaver_legacy_postinstall_done
  ${EndIf}
  ${If} $BeaverLegacyDir != $INSTDIR
    Goto beaver_legacy_postinstall_done
  ${EndIf}

  ReadRegStr $R8 SHCTX "${BEAVER_NEW_UNINSTALL}" "UninstallString"
  ${If} $R8 == ""
    Goto beaver_legacy_postinstall_done
  ${EndIf}
  ReadRegStr $R8 SHCTX "${BEAVER_NEW_PRODUCT}" ""
  ${If} $R8 != $INSTDIR
    Goto beaver_legacy_postinstall_done
  ${EndIf}
  IfFileExists "$INSTDIR\${BEAVER_MAIN_BINARY}" 0 beaver_legacy_postinstall_done

  !insertmacro BeaverDeleteLegacyShortcut "$SMPROGRAMS\CL-GO.lnk"
  !insertmacro BeaverDeleteLegacyShortcut "$SMPROGRAMS\CL-GO\CL-GO.lnk"
  !insertmacro BeaverDeleteLegacyShortcut "$DESKTOP\CL-GO.lnk"
  DeleteRegKey SHCTX "Software\Microsoft\Windows\CurrentVersion\Uninstall\CL-GO"
  DeleteRegKey SHCTX "Software\clgo\CL-GO"
  StrCpy $BeaverLegacyMigration 2

  beaver_legacy_postinstall_done:
  ; Demande à Explorer de relire les icônes après chaque installation/mise à jour.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend
