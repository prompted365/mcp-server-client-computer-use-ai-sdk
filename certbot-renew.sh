#!/bin/sh
certbot renew --webroot -w /var/www/certbot --quiet --agree-tos --no-eff-email --force-renewal
nginx -s reload
