#! /bin/bash
mkdir corctf-challenge-dev-2

# we could also maintain a second copy in this folder, but i dont see a huge problem with copying from the fake challenge
cp -r ../corctf-challenge-dev-2-fake/ccd_site corctf-challenge-dev-2/ccd2_site
cp -r extension_site corctf-challenge-dev-2/extension_site

sed -i "s/18af8d6f5a98ddbe55d16540f174bc16/ADMIN_PASSWORD/g" corctf-challenge-dev-2/extension_site/bot/extension/js/feed_handler.js
sed -i "s/3000/PORT/g" corctf-challenge-dev-2/extension_site/bot/extension/js/feed_handler.js

# fairly certain no files need to be removed? lol
tar -czvf corctf-challenge-dev-2.tar.gz corctf-challenge-dev-2
rm -rf corctf-challenge-dev-2/
