docker-build:
	docker build --tag boa .

docker-container:
	docker create --tty --interactive \
		--name boa \
		--hostname boa \
		--volume ${PWD}/:/usr/src/myapp \
		--publish 9228:9228 \
		boa

docker-clean:
	docker rm boa || echo "no container"
	docker rmi boa || echo "no image"