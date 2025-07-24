ASNRun Scrapper
================

A web scrapper for Strava activities

## Requirements
- Rust 1.85-nightly
- Diesel
- MySQL

## Preparation
1. Install Rust: https://www.rust-lang.org/tools/install
2. Install Rust Nightly: `rustup install nightly`
3. Install Diesel CLI: `cargo install diesel_cli --no-default-features --features mysql`
4. Install MySQL: https://dev.mysql.com/downloads/mysql/
5. Create a MySQL database and user for the application.
6. Install Mysql client libraries:
   - On Ubuntu: `sudo apt-get install libmysqlclient-dev`
   - On macOS: `brew install mysql`
   - On Windows: Use the MySQL installer to include client libraries.
7. Set up the `.env` file with your database connection string:
   ```
   DATABASE_URL=mysql://username:password@localhost/database_name
   USE_PROXY=false
   ```
8. Run the application: `cargo run --bin asnrun-scrapper -- --jobs=4`

## Usage
- The application scrapes Strava activities and stores them in the MySQL database.
- You can adjust the number of jobs by changing the `--jobs` parameter.
- Enter the cookie value for Strava authentication in the `cookies.json` file:
  ```json
  [
    {
        "email": "your_email",
        "cookie": "your_cookie_value"
    }
  ]
  ```
- Ensure that the `cookies.json` file is in the same directory as the executable.
- The application will read the cookies and use them to authenticate with Strava.
- To use the proxy, set `USE_PROXY=true` in the `.env` file.

## Run in Release Mode
To run the application in release mode, use the following command:
```bash
$ cargo build --release
$ cp target/release/asnrun-scrapper .
$ ./asnrun-scrapper --jobs=4
```
This will compile the application with optimizations for better performance.
