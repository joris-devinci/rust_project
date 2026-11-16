API 

Get in the Database:
- docker compose up -d
- docker ps
- docker exec -it {hash_from_the_container} bash
- psql -U user -d db

Stop Database:
- docker compose down (At a '-v' at the end to delete the database contents)

Producer 3.34 sec
Consumer 50-60 sec