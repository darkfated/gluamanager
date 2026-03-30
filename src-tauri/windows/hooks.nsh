!include "LogicLib.nsh"

!macro NSIS_HOOK_POSTINSTALL
  Push $0
  Push $1

  ReadRegStr $0 HKCU "Environment" "Path"
  ${If} $0 == ""
    WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
  ${Else}
    ${If} $0 == "$INSTDIR"
      ; Already present.
    ${Else}
      WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
    ${EndIf}
  ${EndIf}

  System::Call 'user32::SendMessageTimeoutW(i 0xffff, i 0x1A, i 0, w "Environment", i 2, i 5000, *i .r1)'

  Pop $1
  Pop $0
!macroend
