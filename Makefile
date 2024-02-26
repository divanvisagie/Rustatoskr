APP_NAME=rustatoskr

main:
	cargo build --release

# run docker compose on docker-compose.test.yml
test:
	docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit

pushpi:
	ssh heimdallr.local "mkdir -p ~/src/" && rsync -av --progress . heimdallr.local:~/src/$(APP_NAME)

install:
	# stop the service if it already exists
	systemctl stop rustatoskr || true
	systemctl disable rustatoskr || true
	# delete the old service file if it exists
	rm /etc/systemd/system/rustatoskr.service || true
	cp scripts/rustatoskr.service /etc/systemd/system/
