echo "Make sure poetry exists..."
WHERE poetry
IF %ERRORLEVEL% NEQ 0 python -m pip install poetry

python -m poetry install