// Tipos generados por Anti-Gravital ag-dsl v0.1-v0.2.
// NO editar manualmente. Regenerar con `ag generate`.

/** Modelo completo incluyendo campos autogenerados. */
export interface User {
  id: string;
  email: string;
  name: string;
  created_at: string;
  updated_at: string;
  orders?: Order[];
}

/** Cuerpo de la peticion POST para crear un nuevo registro. */
export interface CreateUserRequest {
  email: string;
  name: string;
}

/** Cuerpo de la peticion PUT/PATCH. Todos los campos son opcionales. */
export interface UpdateUserRequest {
  email?: string;
  name?: string;
}

/** Modelo completo incluyendo campos autogenerados. */
export interface Category {
  id: string;
  name: string;
  products?: Product[];
}

/** Cuerpo de la peticion POST para crear un nuevo registro. */
export interface CreateCategoryRequest {
  name: string;
}

/** Cuerpo de la peticion PUT/PATCH. Todos los campos son opcionales. */
export interface UpdateCategoryRequest {
  name?: string;
}

/** Modelo completo incluyendo campos autogenerados. */
export interface Product {
  id: string;
  name: string;
  description?: string | null;
  price: string;
  stock: number;
  category_id: string;
  category?: Category;
  created_at: string;
  order_items?: OrderItem[];
}

/** Cuerpo de la peticion POST para crear un nuevo registro. */
export interface CreateProductRequest {
  name: string;
  description?: string | null;
  price: string;
  stock: number;
  category_id: string;
}

/** Cuerpo de la peticion PUT/PATCH. Todos los campos son opcionales. */
export interface UpdateProductRequest {
  name?: string;
  description?: string;
  price?: string;
  stock?: number;
  category_id?: string;
}

/** Modelo completo incluyendo campos autogenerados. */
export interface Order {
  id: string;
  user_id: string;
  user?: User;
  total: string;
  status: string;
  created_at: string;
  updated_at: string;
  items?: OrderItem[];
}

/** Cuerpo de la peticion POST para crear un nuevo registro. */
export interface CreateOrderRequest {
  user_id: string;
  total: string;
  status: string;
}

/** Cuerpo de la peticion PUT/PATCH. Todos los campos son opcionales. */
export interface UpdateOrderRequest {
  user_id?: string;
  total?: string;
  status?: string;
}

/** Modelo completo incluyendo campos autogenerados. */
export interface OrderItem {
  id: string;
  order_id: string;
  product_id: string;
  order?: Order;
  product?: Product;
  quantity: number;
  unit_price: string;
}

/** Cuerpo de la peticion POST para crear un nuevo registro. */
export interface CreateOrderItemRequest {
  order_id: string;
  product_id: string;
  quantity: number;
  unit_price: string;
}

/** Cuerpo de la peticion PUT/PATCH. Todos los campos son opcionales. */
export interface UpdateOrderItemRequest {
  order_id?: string;
  product_id?: string;
  quantity?: number;
  unit_price?: string;
}

/** API type `CreateUserRequest`. */
export interface CreateUserRequest {
  email: string;
  name: string;
}

/** API type `CreateProductRequest`. */
export interface CreateProductRequest {
  name: string;
  price: string;
  stock: number;
  category_id: string;
}

/** API type `CreateOrderRequest`. */
export interface CreateOrderRequest {
  user_id: string;
  items: string;
}

/** API type `UserResponse`. */
export interface UserResponse {
  id: string;
  email: string;
  name: string;
}

/** API type `ProductResponse`. */
export interface ProductResponse {
  id: string;
  name: string;
  price: string;
  stock: number;
  category_id: string;
}

/** API type `OrderResponse`. */
export interface OrderResponse {
  id: string;
  user_id: string;
  total: string;
  status: string;
  created_at: string;
}

