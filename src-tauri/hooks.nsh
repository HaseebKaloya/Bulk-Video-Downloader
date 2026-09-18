; Modern UI 2 Layout & Size Enhancements
; Enlarge installer window and scale all controls proportionally with modern Segoe UI font
SetFont "Segoe UI" 10

!define MUI_WELCOMEPAGE_TITLE_3LINES
!define MUI_FINISHPAGE_TITLE_3LINES


!macro NSIS_HOOK_POSTINSTALL
  ; 1. Ensure Desktop Shortcut is created with custom brand icon
  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
  
  ; 2. Register Auto Startup on Windows Boot (Task Manager Startup Apps)
  CreateShortcut "$SMSTARTUP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}" '"$INSTDIR\${MAINBINARYNAME}.exe"'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Clean up desktop shortcut, startup shortcut, and registry run entry
  Delete "$DESKTOP\${PRODUCTNAME}.lnk"
  Delete "$SMSTARTUP\${PRODUCTNAME}.lnk"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
!macroend
