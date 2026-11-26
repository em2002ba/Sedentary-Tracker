# Runtime image for the Sedentary Tracker server.
# The binary is compiled in the CI "build" job (which has a Postgres service,
# needed by the sqlx compile-time query checks) and copied in here.
# ubuntu:24.04 matches the glibc of the GitHub ubuntu-latest runner.
FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libudev1 curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 tracker

# The server serves the dashboard from <checkout dir>/server/../frontend, a path
# fixed at compile time. CI passes the runner's checkout dir as SRC_DIR so the
# dashboard files land at exactly that path inside the image.
ARG SRC_DIR=/app
COPY frontend/ ${SRC_DIR}/frontend/
COPY dist/server /usr/local/bin/sedentary-server

USER tracker
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=3s CMD curl -fs http://localhost:8000/health || exit 1
CMD ["sedentary-server"]
