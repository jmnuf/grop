@ECHO OFF
WHERE /q rustc
IF %ERRORLEVEL% NEQ 0 (
   ECHO [ERROR] Missing `rustc` compiler. This is a requirement, install it genius
   EXIT /B 1
)
WHERE /q kinoko
IF %ERRORLEVEL% NEQ 0 (
   ECHO [INFO] `kinoko` not found, defaulting to `rustc`
   ECHO [CMD] rustc .\src\main.rs -o .\build\grop.exe
   rustc src\main.rs -o build\grop.exe
   IF %ERRORLEVEL% EQU 0 (
      ECHO [CMD] .\build\grop.exe -h
      .\build\grop.exe -h
   )
   EXIT /b %ERRORLEVEL%
) ELSE (
  ECHO [CMD] kinoko build -r -- -h
  kinoko build -r -- -h
)
