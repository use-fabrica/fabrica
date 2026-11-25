# Fabrica

Fabrica coordinates execution of application code across workers.

## Language

**Actor**:
A tenant-scoped entity with an identity, durable state, and a mailbox.
An actor processes one message at a time and is not tied to a particular worker.

**Actor Key**:
The application-supplied part of an actor's identity. The same tenant, actor
type, and key identify the same actor, even across separate agent runs.
