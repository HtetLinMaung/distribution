CREATE TABLE users
(
    user_id SERIAL PRIMARY KEY,
    full_name VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(100),
    -- For role-based access control (e.g., 'admin', 'user')
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

insert into users
    (username, password, full_name, role, created_at)
values
    ('distributor', '$2b$12$VsrfBeuszFplm3HX4QgMWOg/KMsIhZgPLCjej2W3DI.YHz9Gq9Zjq', 'Thant Zin', 'Disributor', now()),
    ('admin', '$2b$12$VsrfBeuszFplm3HX4QgMWOg/KMsIhZgPLCjej2W3DI.YHz9Gq9Zjq', 'Thant Zin Lin', 'Admin', now
());


CREATE TABLE townships
(
    township_id SERIAL PRIMARY KEY,
    township_name varchar(50) not null,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);
INSERT INTO townships
    (township_name)
VALUES
    ('Hlaing Township');

CREATE TABLE wards
(
    ward_id SERIAL PRIMARY KEY,
    ward_name varchar(255) not null,
    township_id INTEGER REFERENCES townships(township_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);
INSERT INTO wards
    (ward_name, township_id)
VALUES
    ('Hlaing Ward 1', 1),
    ('Hlaing Ward 2', 1);


CREATE TABLE user_wards
(
    user_id INT REFERENCES users(user_id),
    ward_id INT REFERENCES wards(ward_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null,
    PRIMARY KEY (user_id, ward_id)
);
INSERT INTO user_wards
    (user_id, ward_id)
VALUES
    (1, 1);

CREATE TABLE shops
(
    shop_id SERIAL PRIMARY KEY,
    shop_name VARCHAR(255) NOT NULL,
    address TEXT,
    latitude DECIMAL(9,6),
    longitude DECIMAL(9,6),
    image_url TEXT DEFAULT '',
    ward_id INT REFERENCES wards(ward_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO shops
    (shop_name, address, ward_id)
SELECT
    'Shop ' || id,
    'Yangon, Hlaing Township, Ward 1, No ' || id,
    1
FROM generate_series(1, 140) id;

CREATE TABLE weekdays
(
    weekday_id SERIAL PRIMARY KEY,
    weekday_name VARCHAR(10),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Monday');
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Tuesday');
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Wednesday');
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Thursday');
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Friday');
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Saturday');
INSERT INTO weekdays
    (weekday_name)
VALUES
    ('Sunday');


CREATE TABLE shop_weekdays
(
    shop_id INT REFERENCES shops(shop_id),
    weekday_id INT REFERENCES weekdays(weekday_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL,
    PRIMARY KEY (shop_id, weekday_id)
);
insert into shop_weekdays
    (shop_id, weekday_id)
SELECT shop, 1
END
FROM generate_series
(1, 20) shop;

CREATE TABLE categories
(
    category_id SERIAL PRIMARY KEY,
    category_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE brands
(
    brand_id SERIAL PRIMARY KEY,
    brand_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);
INSERT INTO brands
    (brand_name, created_at, deleted_at)
VALUES
    ('Apple', '2023-01-01 09:00:00', NULL),
    ('Samsung', '2023-01-02 09:30:00', NULL),
    ('Sony', '2023-01-03 09:45:00', NULL),
    ('Dell', '2023-01-04 10:00:00', NULL),
    ('GoPro', '2023-01-05 10:15:00', NULL);


CREATE TABLE products
(
    product_id SERIAL PRIMARY KEY,
    product_name VARCHAR(255) NOT NULL,
    image_url TEXT DEFAULT '',
    brand_id INT REFERENCES brands(brand_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE product_prices
(
    price_id SERIAL PRIMARY KEY,
    product_id INT REFERENCES products(product_id),
    price DECIMAL NOT NULL,
    price_type VARCHAR(50),
    -- e.g., 'single_item', 'package'
    package_quantity INT DEFAULT 1,
    -- Relevant only for package deals
    remaining_quantity INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

INSERT INTO products
    (product_name, image_url, brand_id, created_at, deleted_at)
VALUES
    ('Apple iPhone 13', 'http://example.com/iphone13.jpg', 1, '2023-01-01 10:00:00', NULL),
    ('Samsung Galaxy S22', 'http://example.com/galaxys22.jpg', 2, '2023-01-02 11:00:00', NULL),
    ('Sony WH-1000XM4 Headphones', 'http://example.com/sonyheadphones.jpg', 3, '2023-01-03 12:00:00', NULL),
    ('Dell XPS 13 Laptop', 'http://example.com/dellxps13.jpg', 4, '2023-01-04 13:00:00', NULL),
    ('GoPro HERO9 Black', 'http://example.com/goprohero9.jpg', 5, '2023-01-05 14:00:00', NULL);

INSERT INTO product_prices
    (product_id, price, price_type, package_quantity, remaining_quantity, created_at, deleted_at)
VALUES
    (6, 799.99, 'single_item', 1, 100, '2023-01-01 10:00:00', NULL),
    (6, 7500.00, 'package', 10, 10, '2023-01-01 10:00:00', NULL),
    (7, 999.99, 'single_item', 1, 50, '2023-01-02 11:00:00', NULL),
    (8, 349.99, 'single_item', 1, 75, '2023-01-03 12:00:00', NULL),
    (9, 1200.00, 'single_item', 1, 30, '2023-01-04 13:00:00', NULL),
    (10, 399.99, 'single_item', 1, 40, '2023-01-05 14:00:00', NULL);


CREATE TABLE product_categories
(
    product_id INT REFERENCES products(product_id),
    category_id INT REFERENCES categories(category_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL,
    PRIMARY KEY (product_id, category_id)
);

-- CREATE TABLE product_brands
-- (
--     product_id INT REFERENCES products(product_id),
--     brand_id INT REFERENCES brands(brand_id),
--     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
--     deleted_at TIMESTAMP DEFAULT NULL,
--     PRIMARY KEY (product_id, brand_id, deleted_at)
-- );

CREATE TABLE discounts
(
    discount_id SERIAL PRIMARY KEY,
    discount_name VARCHAR(255) NOT NULL,
    discount_type VARCHAR(50) DEFAULT 'percentage',
    -- e.g., 'percentage', 'fixed_amount'
    discount_value DECIMAL DEFAULT 0.0,
    -- Percentage or fixed amount
    start_date DATE,
    end_date DATE,
    min_quantity INT,
    -- Minimum quantity required for the discount to apply
    max_quantity INT,
    -- Maximum quantity up to which the discount applies
    conditions TEXT,
    -- Any additional conditions or notes
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE product_discounts
(
    price_id INT REFERENCES product_prices(price_id),
    discount_id INT REFERENCES discounts(discount_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL,
    PRIMARY KEY (price_id, discount_id, deleted_at)
);


CREATE TABLE orders
(
    order_id SERIAL PRIMARY KEY,
    shop_id INT REFERENCES shops(shop_id),
    user_id INT REFERENCES users(user_id) NULL,
    -- Optional, if orders are linked to specific users
    order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delivery_date DATE,
    -- When the order is scheduled for delivery
    status VARCHAR(50),
    -- e.g., 'Pending', 'Delivered'
    total_amount DECIMAL,
    -- Total amount of the order, can be calculated in the application layer
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);


CREATE TABLE order_details
(
    order_detail_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES orders(order_id),
    price_id INT REFERENCES product_prices(price_id),
    quantity INT NOT NULL,
    price_at_order DECIMAL,
    -- Price of the product at the time of ordering
    discount_id INT REFERENCES discounts(discount_id) NULL,
    -- Optional, if a discount is applied
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);


-- additional tables

-- If you plan to handle multiple distribution centers or warehouses, a table for them can be useful.
CREATE TABLE distribution_centers
(
    center_id SERIAL PRIMARY KEY,
    center_name VARCHAR(255) NOT NULL,
    address TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- To track the inventory of products at different distribution centers.
CREATE TABLE inventory
(
    inventory_id SERIAL PRIMARY KEY,
    product_id INT REFERENCES products(product_id),
    center_id INT REFERENCES distribution_centers(center_id),
    quantity INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- If your distribution system involves a fleet of vehicles, this table can track them.
CREATE TABLE delivery_vehicles
(
    vehicle_id SERIAL PRIMARY KEY,
    vehicle_type VARCHAR(100),
    license_plate VARCHAR(50),
    center_id INT REFERENCES distribution_centers(center_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- If you decide to reintroduce the concept of employees or distributors, especially those handling logistics.
CREATE TABLE employees
(
    employee_id SERIAL PRIMARY KEY,
    employee_name VARCHAR(255) NOT NULL,
    job_title VARCHAR(100),
    center_id INT REFERENCES distribution_centers(center_id),
    vehicle_id INT REFERENCES delivery_vehicles(vehicle_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- To provide detailed tracking of each order's journey.
CREATE TABLE order_tracking
(
    tracking_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES orders(order_id),
    status VARCHAR(50),
    location TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- If you need to manage information about where your products are coming from.
CREATE TABLE suppliers
(
    supplier_id SERIAL PRIMARY KEY,
    supplier_name VARCHAR(255),
    contact_info TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

-- If products can come from multiple suppliers.
CREATE TABLE product_suppliers
(
    product_id INT REFERENCES products(product_id),
    supplier_id INT REFERENCES suppliers(supplier_id),
    PRIMARY KEY (product_id, supplier_id)
);
