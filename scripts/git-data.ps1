git describe --tags --abbrev=0 > VERSION
git rev-parse HEAD > GIT_SHA
git rev-parse --abbrev-ref HEAD > GIT_BRANCH