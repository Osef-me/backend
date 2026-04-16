FROM postgres:17-alpine

# All migration files run in filename order at first start
COPY migrations/ /docker-entrypoint-initdb.d/
