FROM ubuntu:24.04

RUN apt-get update && \
    apt-get install -y nodejs && \
    apt-get install -y npm

RUN apt-get install -y build-essential && \
    apt-get install -y curl && \
    apt-get install -y gh

RUN npm i -g @openai/codex

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"