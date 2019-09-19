FROM rust:latest

WORKDIR /usr/src/myapp
COPY . .

# LLDB Server
EXPOSE 9228

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get install -y sudo software-properties-common libpython2.7

# codelldb depends on libpython2.7
# https://stackoverflow.com/questions/20842732/libpython2-7-so-1-0-cannot-open-shared-object-file-no-such-file-or-directory

# https://askubuntu.com/questions/787383/how-to-install-llvm-3-9
# http://apt.llvm.org/
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-add-repository "deb http://apt.llvm.org/stretch/ llvm-toolchain-stretch-6.0 main"
RUN apt-get -y update
RUN sudo apt-get install -y lldb


CMD ["/bin/bash"]