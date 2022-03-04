FROM python:3.8

RUN pip install pipenv

WORKDIR /app/

COPY Pipfile ./
COPY Pipfile.lock ./
RUN pipenv install

COPY ./lithicrivers/ /app/lithicrivers/
# EXPOSE 5000

CMD ["pipenv", "run", "python", "-m", "lithicrivers"]
