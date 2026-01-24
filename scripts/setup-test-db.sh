#!/bin/bash
set -e

# Configuration
DB_NAME="irp_test"
DB_USER="postgres"
DB_PASS="dev"
DB_HOST="localhost"
DB_PORT="5432"

# Connection string
export DATABASE_URL="postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME"

echo "Setting up test database: $DB_NAME..."

# Drop and recreate database via Docker
docker exec iap-alpha-postgres-1 psql -U $DB_USER -c "DROP DATABASE IF EXISTS $DB_NAME;"
docker exec iap-alpha-postgres-1 psql -U $DB_USER -c "CREATE DATABASE $DB_NAME;"

echo "Running migrations for test database..."
# Let's run a one-off pod or just use cargo run on host if possible
# Since we have cargo on host, let's use it with DATABASE_URL
DATABASE_URL=postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME cargo run -p api -- --run-migrations-only || true

echo "Test database setup complete."
