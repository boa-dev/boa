FROM rust:latest

WORKDIR /usr/src/myapp

RUN apt-get -y update && apt-get -y upgrade

CMD ["/bin/bash"]