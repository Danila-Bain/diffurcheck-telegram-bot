# Diffurcheck

This project is a Telegram bot designed to conduct remote written tests 
with the course on differential equations in mind. Implemented in
`Rust` for bot functionality and `Typst` for rendering problems.

### Purpose and functionality

- After registration, a student can choose one of the available assignments.
  Each assignment can be started at any convenient time before the specified
  deadline. Once started, a fixed amount of time is allocated to complete it.

- Upon starting an assignment, the student receives the problems along with the
  exact submission deadline. Solutions must be submitted before this time as
  scanned handwritten work, either as a single PDF file or as image files
  uploaded as documents.

- After the overall deadline, all submitted solutions are automatically
  compiled into a single PDF file that also includes the assignment variants
  and the correct answers. The instructor reviews the solutions, annotates
  errors and comments, and then splits the reviewed file back into individual
  reports. Each student receives their checked work along with the assessment
  results. (For now, splitting functionality is not yet implemented)

# Setup

## Environment file (not included)
In rust the call `dotenvy::dotenv().ok();` loads environment variables from `.env` file, which,
in my case, has contents like
```
DATABASE_URL=postgres://postgres:diffur27@localhost:5432/diffurcheck_bot
TELOXIDE_TOKEN=123456789:AJLDKOASjojdoaisdjASDjoiASDo_12j90jaAL:wK
ADMIN_CHAT_ID=-1234567890
RUST_LOG=debug
```
The variable `DATABASE_URL` is also used by `sqlx` commands to connect to database server.

## Database installation

For `arch-linux`, I used the following commands
```
sudo pacman -Syu postgresql
sudo -iu postgres initdb --locale ru_RU.UTF-8 -D /var/lib/postgres/data

sudo systemctl enable postgresql.service
sudo systemctl start postgresql.service


cargo install sqlx-cli --no-default-features --features postgres,rustls
```

## Database management
Then, to create new database:
```
cargo sqlx database create
```

To apply new migrations:
```
cargo sqlx migrate run
```  

To nuke current database and reapply all migrations:
```
cargo sqlx database reset
```

## Run

Run with `cargo run` or `cargo run --release`. Release versions tend to compile for
too long for fast development iteration.

# Usage

## Managing assignment generators


To add new assignment generator, from `assignment/`:
```
cargo new my_assignment
```
this way, cargo will automatically add this crate as a workspace member.

It is usefull to add the following dependencies for typst generation
```
cargo add typst --features=typst-kit-fonts,typst-kit-embed-fonts
cargo add typst_render image typst-pdf typst-png typst-as-lib
cargo add serde, serde_json --features=derive
```

To run assignment generator, inside corresponding crate run
```
export RUST_LOG=debug; cargo run >> /null/dev
```
to see generated images in debug output. After command runs, press `<CTRL-D>` 
for `EOF` character. This will cause parse error for incoming json, forcing
generator to use some default dummy data for variant number.

