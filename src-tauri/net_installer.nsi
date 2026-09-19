; Bulk Video Downloader - Net Installer (Online Bootstrapper Edition)
; Author: Haseeb Kaloya (https://github.com/HaseebKaloya)
; Architecture: x64 Windows 10/11

Unicode true
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "x64.nsh"

# General Definitions
!define PRODUCT_NAME "Bulk Video Downloader"
!define PRODUCT_EDITION "Network Installer"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "Haseeb Kaloya"
!define PRODUCT_WEB_SITE "https://github.com/HaseebKaloya"
!define MAIN_BINARY_NAME "Bulk Video Downloader.exe"
!define UNINSTALL_NAME "uninstall.exe"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION} (${PRODUCT_EDITION})"
OutFile "..\..\releases\Bulk-Video-Downloader_1.0.0_x64_NetInstaller.exe"
InstallDir "$PROGRAMFILES64\Bulk Video Downloader"
InstallDirRegKey HKLM "Software\${PRODUCT_NAME}" "InstallDir"
RequestExecutionLevel admin

; UI Configuration
SetFont "Segoe UI" 10
!define MUI_ICON "icons\icon.ico"
!define MUI_UNICON "icons\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "installer-sidebar.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "installer-sidebar.bmp"
!define MUI_ABORTWARNING

; Welcome Page
!define MUI_WELCOMEPAGE_TITLE "Welcome to ${PRODUCT_NAME} Setup"
!define MUI_WELCOMEPAGE_TEXT "This lightweight network installer will setup ${PRODUCT_NAME} on your system.$\r$\n$\r$\nCore runtime components will be installed immediately. The native yt-dlp extraction engine and FFmpeg media processors will be automatically retrieved and updated via high-speed CDN on first run.$\r$\n$\r$\nClick Next to continue."
!insertmacro MUI_PAGE_WELCOME

; License Page
!insertmacro MUI_PAGE_LICENSE "..\LICENSE.txt"

; Directory Page
!insertmacro MUI_PAGE_DIRECTORY

; Installation Files Page
!insertmacro MUI_PAGE_INSTFILES

; Finish Page
!define MUI_FINISHPAGE_RUN "$INSTDIR\${MAIN_BINARY_NAME}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch ${PRODUCT_NAME}"
!define MUI_FINISHPAGE_LINK "Visit Haseeb Kaloya's GitHub Profile"
!define MUI_FINISHPAGE_LINK_LOCATION "https://github.com/HaseebKaloya"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "MainSection" SEC01
  SectionIn RO
  SetOutPath "$INSTDIR"
  
  ; Install Main Executable & License
  File "/oname=${MAIN_BINARY_NAME}" "target\release\bulk-video-downloader.exe"
  File "..\LICENSE.txt"
  
  ; Create local bin folder for network-downloaded yt-dlp & ffmpeg
  CreateDirectory "$INSTDIR\bin"

  ; Create Uninstaller
  WriteUninstaller "$INSTDIR\${UNINSTALL_NAME}"

  ; Registry Keys for Add/Remove Programs
  WriteRegStr HKLM "Software\${PRODUCT_NAME}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "DisplayName" "${PRODUCT_NAME} (Net Edition)"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "DisplayIcon" "$INSTDIR\${MAIN_BINARY_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "UninstallString" '"$INSTDIR\${UNINSTALL_NAME}"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"

  ; Desktop & Start Menu Shortcuts
  CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
  CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" "$INSTDIR\${MAIN_BINARY_NAME}" "" "$INSTDIR\${MAIN_BINARY_NAME}" 0
  CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall ${PRODUCT_NAME}.lnk" "$INSTDIR\${UNINSTALL_NAME}" "" "$INSTDIR\${UNINSTALL_NAME}" 0
  CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\${MAIN_BINARY_NAME}" "" "$INSTDIR\${MAIN_BINARY_NAME}" 0
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall ${PRODUCT_NAME}.lnk"
  RMDir "$SMPROGRAMS\${PRODUCT_NAME}"

  Delete "$INSTDIR\${MAIN_BINARY_NAME}"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\${UNINSTALL_NAME}"
  RMDir /r "$INSTDIR\bin"
  RMDir "$INSTDIR"

  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
  DeleteRegKey HKLM "Software\${PRODUCT_NAME}"
SectionEnd
