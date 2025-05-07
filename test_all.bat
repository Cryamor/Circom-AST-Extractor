@echo off
for /l %%i in (1,1,23) do (
    if exist "testcase\%%i.circom" (
        main.exe "testcase\%%i.circom"
    ) else (
        echo File Not Exist
    )
)