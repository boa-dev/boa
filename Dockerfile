FROM rust:latest

WORKDIR /usr/src/myapp
COPY . .

# LLDB Server
EXPOSE 9228

RUN apt-get update \
    apt-get upgrade \
    apt-get install -y software-properties-common

# https://askubuntu.com/questions/787383/how-to-install-llvm-3-9
# http://apt.llvm.org/
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-add-repository "deb http://apt.llvm.org/stretch/ llvm-toolchain-stretch-6.0 main"
RUN apt-get update
RUN apt-get install clang-6.0 lldb-6.0 


CMD ["/bin/bash"]