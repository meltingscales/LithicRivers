FROM python:3.8

RUN pip install poetry

WORKDIR /app/

COPY pyproject.toml ./
COPY Pipfile.lock ./
RUN pipenv install

COPY ./lithicrivers/ /app/lithicrivers/
# EXPOSE 5000

CMD ["pipenv", "run", "python", "-m", "lithicrivers"]
