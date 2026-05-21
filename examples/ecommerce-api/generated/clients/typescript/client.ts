// Cliente HTTP generado por Anti-Gravital ag-dsl v0.2.
// NO editar manualmente. Regenerar con `ag generate`.

const BASE_URL = '';

/** GET /users */
export async function listUsers() : Promise<UserResponse> {
  const resp = await fetch(`${BASE_URL}/users`, {
    method: 'get',
  });
  if (!resp.ok) throw new Error(`listUsers failed: ${resp.status}`);
  return resp.json();
}

/** GET /users/{id} */
export async function getUser(id: string) : Promise<UserResponse> {
  const resp = await fetch(`${BASE_URL}/users/${id}`, {
    method: 'get',
  });
  if (!resp.ok) throw new Error(`getUser failed: ${resp.status}`);
  return resp.json();
}

/** POST /users */
export async function createUser(body: CreateUserRequest) : Promise<UserResponse> {
  const resp = await fetch(`${BASE_URL}/users`, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`createUser failed: ${resp.status}`);
  return resp.json();
}

/** GET /products */
export async function listProducts() : Promise<ProductResponse> {
  const resp = await fetch(`${BASE_URL}/products`, {
    method: 'get',
  });
  if (!resp.ok) throw new Error(`listProducts failed: ${resp.status}`);
  return resp.json();
}

/** GET /products/{id} */
export async function getProduct(id: string) : Promise<ProductResponse> {
  const resp = await fetch(`${BASE_URL}/products/${id}`, {
    method: 'get',
  });
  if (!resp.ok) throw new Error(`getProduct failed: ${resp.status}`);
  return resp.json();
}

/** POST /products */
export async function createProduct(body: CreateProductRequest) : Promise<ProductResponse> {
  const resp = await fetch(`${BASE_URL}/products`, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`createProduct failed: ${resp.status}`);
  return resp.json();
}

/** POST /orders */
export async function createOrder(body: CreateOrderRequest) : Promise<OrderResponse> {
  const resp = await fetch(`${BASE_URL}/orders`, {
    method: 'post',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`createOrder failed: ${resp.status}`);
  return resp.json();
}

/** GET /orders/{id} */
export async function getOrder(id: string) : Promise<OrderResponse> {
  const resp = await fetch(`${BASE_URL}/orders/${id}`, {
    method: 'get',
  });
  if (!resp.ok) throw new Error(`getOrder failed: ${resp.status}`);
  return resp.json();
}

