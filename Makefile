# run docker compose on docker-compose.test.yml
test:
	docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit