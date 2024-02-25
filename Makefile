APP_NAME=rustatoskr
# run docker compose on docker-compose.test.yml
test:
	docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit

pushpi:
	ssh heimdallr.local "mkdir -p ~/src/" && rsync -av --progress . heimdallr.local:~/src/$(APP_NAME)
