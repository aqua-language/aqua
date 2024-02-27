docker image rm -f aqua aqua-nexmark &>/dev/null && echo 'Removed old image'
docker container rm -f aqua-nexmark &>/dev/null && echo 'Removed old container'
docker build --tag aqua -f ../../Dockerfile ../../
docker build --tag aqua-nexmark .
docker run --name aqua-nexmark aqua-nexmark
docker cp aqua-nexmark:/aqua/experiments/nexmark/output .
