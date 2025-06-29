import { User, Post, Status } from '../types/api';

export async function fetchUsers(): Promise<User[]> {
  const response = await fetch('/api/users');
  return response.json();
}

export async function fetchPosts(): Promise<Post[]> {
  const response = await fetch('/api/posts');
  return response.json();
}

export function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString();
}

export function unusedHelper(value: string): string {
  return value.toUpperCase();
}

export const USED_CONSTANT = 'This constant is used';
export const UNUSED_CONSTANT = 'This constant is never used';

export function calculateTotal(items: number[]): number {
  return items.reduce((sum, item) => sum + item, 0);
}