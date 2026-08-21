# Production Architecture

## Deployment Boundary

External clients

-> API Layer

-> Application Layer

-> Domain Core

## Infrastructure Responsibilities

Infrastructure supports deployment, validation, configuration, and operations.

Infrastructure does not own:

- translation logic
- literary intelligence
- memory mutation
- quality decisions
- human review decisions

## Environment Strategy

Supported environments:

- development
- testing
- production

Configuration is supplied through environment variables and configuration files.

## CI/CD Flow

Pull requests validate formatting, linting, and tests.

Main branch validates release builds and container builds.

## Scaling Preparation

Future scaling may introduce workers and separated processing boundaries without changing domain ownership.
