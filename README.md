# full-stack-actix-web-crud

Buidling a full stack CRUD using Actix Web Rust and VueJs as the web UI

## Tech Stacks

- Backend
  - Actix Web (Rust) [Actix Web](https://actix.rs/)
  - SQL Server for Database [MSSQL](https://www.microsoft.com/en-us/sql-server/sql-server-downloads)
    - For connect to DB using [tiberius](https://docs.rs/tiberius/latest/tiberius/)
  - Structure as a Features Base
  - Jwt as an authentication method [jwt.io](https://jwt.io/)
- Web
  - [VueJs](https://vuejs.org/)

## Features

- <b>`Authentication`</b>
  - Register user
  - Login using JWT for generation token also support cookie
  - Logut
- <b>`Users`</b>
  - Get All Users
  - Get User by Id
- <b>`Roles`</b>
  - Get all Roles
  - Get User's Roles
  - Create new Role
  - Update Role
  - Assing Role to User
