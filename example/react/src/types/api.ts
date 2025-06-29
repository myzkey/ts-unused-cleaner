export interface User {
  id: number;
  name: string;
  email: string;
  avatar?: string;
}

export interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

export type UserListResponse = ApiResponse<User[]>;

export interface Post {
  id: number;
  title: string;
  content: string;
  authorId: number;
  createdAt: string;
}

export type UnusedDataType = {
  id: string;
  value: number;
  metadata: Record<string, unknown>;
};

export enum Status {
  PENDING = 'pending',
  LOADING = 'loading',
  SUCCESS = 'success',
  ERROR = 'error'
}

export enum UnusedStatus {
  DRAFT = 'draft',
  PUBLISHED = 'published',
  ARCHIVED = 'archived'
}