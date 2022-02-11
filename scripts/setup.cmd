echo "Make sure pipenv exists..."
WHERE poetry
IF %ERRORLEVEL% NEQ 0 python3 -m pip install pipenv

python3 -m pipenv install --dev