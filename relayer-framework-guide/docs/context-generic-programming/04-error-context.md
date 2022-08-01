# Error Context

One thing to note however is that the `Error` type in `QueryPersonContext`
is concrete. With that, it would be problematic if we want to define new contexts
that have different query methods but also return different errors. While it
is possible to define a dynamic error type such as `Box<dyn Error>`, such type
erasure would mean that we lose information about what kinds of errors can happen
when we try to query for `Person` details.

We can instead make the error type _generic_. But instead of using it as a
generic parameter for `greet`, we can define it as an _associated type_ for
the generic type `Context`:

```rust
# struct PersonId(String);
# struct Person {
#   id: PersonId,
#   name: String,
# }
#
trait ErrorContext {
  type Error;
}

trait QueryPersonContext: ErrorContext {
  fn query_person(&self, person_id: &PersonId) ->  Result<Person, Self::Error>;
}

fn greet<Context>(context: &Context, person_id: &PersonId)
  -> Result<(), Context::Error>
where
  Context: QueryPersonContext,
{
  let person = context.query_person(person_id)?;
  println!("Hello, {}", person.name);
  Ok(())
}
```

We define a new `ErrorContext` trait with only one thing, which is the `Error`
associated type. Aside from that, there is nothing known about the `Error`
type, but that is ok as we will see later on. The trait `QueryPersonContext`
then has `ErrorContext` as its supertrait, esentially allowing it to access
the associated type as `Self::Error` in the return type of `query_person`.

We define the `Error` associated type in a separate `ErrorContext` trait,
instead of directly in the `QueryPersonContext` trait. As we will see later,
this is essential to allow multiple context traits to access the same
`Error` type.

In the `greet` function, we require the generic `Context` type to
implement `QueryPersonContext`. But since `ErrorContext` is a supertrait of
`QueryPersonContext`, we would also able to access the error type as
`Context::Error`.