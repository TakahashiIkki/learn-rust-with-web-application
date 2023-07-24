import { Stack, Typography } from "@mui/material";
import { FC } from "react";
import { Todo } from "../types/todo";
import TodoItem from "./TodoItem.tsx";

type Props = {
  todos: Todo[];
  onUpdate: (todo: Todo) => void;
  onDelete: (id: number) => void;
};

const TodoList: FC<Props> = ({ todos, onUpdate, onDelete }) => {
  return (
    <Stack spacing={2}>
      <Typography variant="h2">Todo list</Typography>
      <Stack spacing={2}>
        {todos.map((todo) => (
          <TodoItem todo={todo} onUpdate={onUpdate} onDelete={onDelete} />
        ))}
      </Stack>
    </Stack>
  );
};

export default TodoList;
