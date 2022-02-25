echo "Make sure pipenv exists..."
WHERE poetry
IF %ERRORLEVEL% NEQ 0 python -m pip install pipenv

python -m pipenv install --dev