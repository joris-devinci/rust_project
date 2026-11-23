# How to start the program

## Start the database

Run the following commands to start up the DB
`docker compose up -d`

Then to enter in the DB do the following
`docker ps`
`docker exec -it {hash_from_the_container\name_from_the_container} bash`
`psql -U user -d db`


## Start the Application

Run `cargo run`

to quit just press ctrl-c


## Stop the database or delete it

Stop Database:
`docker compose down`

Stop and delete volume(content) of the database:
`docker compose down -v`