#!/bin/sh

echo "Installing pre-commit hooks..."

BASEDIR=$(dirname "$0")
cp $BASEDIR/hooks/pre-commit $BASEDIR/.git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "Done!"
