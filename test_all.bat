@echo off

if exist "./target/debug/Circom_AST_Extractor.exe" (
    copy /Y ".\target\debug\Circom_AST_Extractor.exe" .

    for /l %%i in (1,1,30) do (
        if exist "testcase\%%i.circom" (
            echo processing "%%i.circom"
            Circom_AST_Extractor.exe "testcase\%%i.circom"
        ) else (
            echo File "testcase\%%i.circom" Not Exist
        )
    )

) else (
    echo File "Circom_AST_Extractor.exe" Not Exist
)

