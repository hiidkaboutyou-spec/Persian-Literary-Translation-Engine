# Production Architecture

## Deployment Boundary

External clients

-> API Layer

-> Application Layer

-> Domain Core

## Infrastructure Responsibilities

Infrastructure supports:

- deployment
- validation
- configuration
- observability
- operations

Infrastructure does not own:

- translation logic
- literary intelligence
- memory mutation
- quality decisions
- human review decisions

## Container Boundary

Production containers should:

- run with least privilege
- expose only required ports
- receive configuration through environment variables
- keep runtime concerns separate from domain behavior

## Environment Strategy

Supported environments:

- development
- testing
- production

Configuration is supplied through environment variables and configuration files.

## CI/CD Flow

Pull requests validate:

- formatting
- linting
- tests

Main branch validates:

- release builds
- container builds

## Scaling Preparation

Future scaling may introduce workers and separated processing boundaries without changing domain ownership.
