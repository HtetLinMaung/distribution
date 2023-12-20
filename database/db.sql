CREATE TABLE brands
(
    id SERIAL PRIMARY KEY,
    brandname VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);


CREATE TABLE categories
(
    id SERIAL PRIMARY KEY,
    categorname VARCHAR(255) NOT NULL,
    description TEXT,
    brand_id INTEGER REFERENCES brands(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);


CREATE TABLE brand_categories
(
    id SERIAL PRIMARY KEY,
    brand_id INTEGER REFERENCES brands(id),
    category_id INTEGER REFERENCES categories(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE users
(
    id SERIAL PRIMARY KEY,
    name varchar(255) not null,
    username VARCHAR(100) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    role_id INTEGER REFERENCES roles(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE roles
(
    id SERIAL PRIMARY KEY,
    role_name VARCHAR(50) NOT NULL UNIQUE,
    deleted_at TIMESTAMP DEFAULT null
);