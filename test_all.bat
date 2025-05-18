@echo off

if exist "./target/debug/main.exe" (
    copy /Y "./target/debug/main.exe" .
) else (
    echo File "main.exe" Not Exist
)

for /l %%i in (1,1,30) do (
    if exist "testcase\%%i.circom" (
        echo processing "%%i.circom"
        main.exe "testcase\%%i.circom"
    ) else (
        echo File "testcase\%%i.circom" Not Exist
    )
)