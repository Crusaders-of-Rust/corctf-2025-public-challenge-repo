FROM redis:7-alpine
CMD ["redis-server", "--maxmemory 512M", "--maxmemory-policy", "allkeys-lru"]