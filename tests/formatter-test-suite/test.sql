-- ── CASE 1: CREATE TABLE — mixed casing and spacing ──────────────────────
CREATE TABLE  users (
    id          SERIAL   PRIMARY KEY,
    name        VARCHAR(255)   NOT NULL,
    email       VARCHAR(255)   UNIQUE NOT NULL,
    age         INTEGER CHECK(age >= 0),
    created_at  TIMESTAMP   DEFAULT   NOW(),
    is_active   BOOLEAN   DEFAULT TRUE
);

CREATE TABLE orders (
    id          SERIAL PRIMARY KEY,
    user_id     INTEGER REFERENCES users(id) ON DELETE CASCADE,
    total       DECIMAL(10,2)   NOT NULL,
    status      VARCHAR(50)     DEFAULT 'pending',
    created_at  TIMESTAMP       DEFAULT NOW()
);

-- ── CASE 2: SELECT with joins — indentation ────────────────────────────────
SELECT
    u.id,
    u.name,
    u.email,
    COUNT(o.id) AS order_count,
    SUM(o.total) AS total_spent,
    MAX(o.created_at) AS last_order_date
FROM users u
    LEFT JOIN orders o ON u.id = o.user_id
WHERE u.is_active = TRUE
    AND u.created_at >= NOW() - INTERVAL '1 year'
GROUP BY
    u.id, u.name, u.email
HAVING COUNT(o.id) > 0
ORDER BY total_spent DESC
LIMIT 100;

-- ── CASE 3: Subquery ───────────────────────────────────────────────────────
SELECT *
FROM users
WHERE id IN (
    SELECT DISTINCT user_id
    FROM orders
    WHERE status = 'completed'
        AND total > 100.00
);

-- ── CASE 4: CTE ──────────────────────────────────────────────────────────
WITH high_value_users AS (
    SELECT user_id, SUM(total) AS lifetime_value
    FROM orders
    WHERE status = 'completed'
    GROUP BY user_id
    HAVING SUM(total) > 1000
),
recent_orders AS (
    SELECT * FROM orders
    WHERE created_at >= NOW() - INTERVAL '30 days'
)
SELECT u.name, hv.lifetime_value, COUNT(ro.id) AS recent_count
FROM users u
JOIN high_value_users hv ON u.id = hv.user_id
LEFT JOIN recent_orders ro ON u.id = ro.user_id
GROUP BY u.name, hv.lifetime_value;

-- ── CASE 5: INSERT and UPDATE ─────────────────────────────────────────────
INSERT INTO users (name, email, age) VALUES
    ('Alice', 'alice@example.com', 30),
    ('Bob',   'bob@example.com',   25),
    ('Carol', 'carol@example.com', 35);

UPDATE users
SET
    is_active   = FALSE,
    name        = TRIM(name)
WHERE email LIKE '%@deprecated.com'
    OR created_at < NOW() - INTERVAL '2 years';

-- ── CASE 6: Index and function ────────────────────────────────────────────
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);

CREATE OR REPLACE FUNCTION get_user_stats(user_id INTEGER)
RETURNS TABLE(order_count BIGINT, total_spent DECIMAL) AS $$
BEGIN
    RETURN QUERY
        SELECT COUNT(*)::BIGINT, COALESCE(SUM(total), 0)
        FROM orders
        WHERE orders.user_id = get_user_stats.user_id;
END;
$$ LANGUAGE plpgsql;
