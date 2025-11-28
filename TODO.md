# TODO

New system, delete all rust bindings to python and javascript.

Instead they must download the container.

- `docker pull ghcr.io/hlop3z/dinja:0.4.3`

Run the rust server and connect to it.

## Python

`from dinja import Renderer, Input`

## Typescript

`import { Renderer, RenderInput, RenderResult } from '@dinja/core';`

also rename imports to

`import { Renderer, Input, Result } from '@dinja/core';`

Remove settings of the service, since now it would be in rust via a docker and not their bindings like before.

Use http in both python and javascript to call the service instead.
