FROM ghcr.io/puppeteer/puppeteer:22

USER root

WORKDIR /app

ENV PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
ENV PUPPETEER_EXECUTABLE_PATH=/usr/bin/google-chrome

COPY package.json package-lock.json ./
RUN npm i

COPY . .

RUN useradd -ms /bin/bash user
#USER user

CMD ["node", "index.js"]