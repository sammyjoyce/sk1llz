# OpenAPI Contract Template — Lane Style

This reference contains the annotated OpenAPI 3.1 template embodying Lane's philosophy.
Load this when **creating a new API spec from scratch**.

## Key Principles in Practice

- Every `description` is prose a non-technical stakeholder can read
- `info.contact` is never omitted — it anchors the product to a team
- `tags` map 1:1 to business capabilities, not code modules
- `operationId` values follow `verbNoun` (e.g., `listBooks`) for codegen stability
- `$ref` everything that appears more than once; shared schemas live in `components`
- Pricing and rate-limit pages are machine-readable via `x-api-evangelist-*` extensions

## Annotated Template

```yaml
openapi: 3.1.0
info:
  title: <ProductName> API                   # Product name, not team name
  version: 1.0.0
  description: |                              # Write for the business reader FIRST
    The central interface for <ecosystem>.
    This API allows partners to <core value proposition>.

    ## Authentication
    All requests require a valid bearer token in the `Authorization` header.
  contact:                                    # NEVER omit — anchors ownership
    name: API Governance Team
    email: api@example.com
    url: https://developer.example.com/support
  license:
    name: Apache 2.0
    url: https://www.apache.org/licenses/LICENSE-2.0.html
  x-api-evangelist-pricing:                   # Lane-specific: machine-readable pricing
    url: https://developer.example.com/pricing
  x-api-evangelist-rate-limits:               # Lane-specific: machine-readable rate limits
    url: https://developer.example.com/rate-limits

tags:
  - name: Inventory                           # Tags = business capabilities
    description: Operations related to book stock and warehouses.
  - name: Orders
    description: Lifecycle management for customer orders.

servers:
  - url: https://api.example.com/v1           # Version in server URL, NOT in paths
    description: Production
  - url: https://sandbox.example.com/v1
    description: Sandbox for integration testing

paths:
  /books:
    get:
      summary: List all books
      operationId: listBooks                  # Stable across codegen targets
      tags: [Inventory]
      description: |
        Retrieve a paginated list of books.
        Supports filtering by author, genre, and publication date.
      parameters:
        - name: limit
          in: query
          description: Maximum number of items to return.
          schema:
            type: integer
            default: 20
            maximum: 100
        - name: cursor
          in: query
          description: Opaque cursor for keyset pagination. Prefer over page numbers.
          schema:
            type: string
      responses:
        '200':
          description: A paginated list of books.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BookList'
        '429':
          $ref: '#/components/responses/RateLimited'
    post:
      summary: Add a new book
      operationId: createBook
      tags: [Inventory]
      description: Register a new book. Requires `write:inventory` scope.
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/BookInput'
            example:                           # Always include a realistic example
              isbn: "978-3-16-148410-0"
              title: "The API Design Guide"
              author_id: "a1b2c3d4"
              price: 2999
      responses:
        '201':
          description: Book created. `Location` header contains the new resource URL.
          headers:
            Location:
              description: URL of the created resource.
              schema:
                type: string
                format: uri
        '400':
          $ref: '#/components/responses/BadRequest'
        '409':
          $ref: '#/components/responses/Conflict'

components:
  schemas:
    BookInput:                                # Separate input from output schema
      type: object
      required: [isbn, title, author_id]
      properties:
        isbn:
          type: string
          pattern: '^(?=(?:\D*\d){10}(?:(?:\D*\d){3})?$)[\d-]+$'
          example: "978-3-16-148410-0"
        title:
          type: string
          example: "The API Design Guide"
        author_id:
          type: string
          format: uuid
        price:
          description: Price in the smallest currency unit (e.g., cents).
          type: integer
          minimum: 0

    Book:
      allOf:
        - $ref: '#/components/schemas/BookInput'
        - type: object
          properties:
            id:
              type: string
              format: uuid
              readOnly: true
            created_at:
              type: string
              format: date-time
              readOnly: true

    BookList:
      type: object
      properties:
        data:
          type: array
          items:
            $ref: '#/components/schemas/Book'
        pagination:
          $ref: '#/components/schemas/CursorPagination'

    CursorPagination:
      type: object
      properties:
        next_cursor:
          type: string
          nullable: true
          description: Pass as `cursor` parameter for the next page. Null if last page.
        has_more:
          type: boolean

    ProblemDetails:                            # RFC 9457 (was 7807)
      type: object
      properties:
        type:
          type: string
          format: uri
          description: A URI reference identifying the problem type.
        title:
          type: string
        status:
          type: integer
        detail:
          type: string
        instance:
          type: string
          format: uri

  responses:
    BadRequest:
      description: Invalid request syntax or validation failure.
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/ProblemDetails'
    RateLimited:
      description: Rate limit exceeded.
      headers:
        Retry-After:
          description: Seconds until the rate limit resets.
          schema:
            type: integer
    Conflict:
      description: Resource already exists (e.g., duplicate ISBN).
      content:
        application/problem+json:
          schema:
            $ref: '#/components/schemas/ProblemDetails'
```

## Checklist Before Committing a Spec

- [ ] Every `operationId` is unique and follows `verbNoun`
- [ ] Every `description` is understandable by a product manager
- [ ] `info.contact` is present with a real team email
- [ ] All reusable schemas are in `components/schemas`
- [ ] Error responses use `application/problem+json`
- [ ] Pagination uses cursors (not page numbers) for non-trivial datasets
- [ ] Input schemas are separated from output schemas (no `readOnly` hacks)
- [ ] At least one `example` per request body and non-trivial response
- [ ] Version lives in `servers[].url`, not in path segments
