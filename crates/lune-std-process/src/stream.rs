use bstr::BString;
use futures_lite::prelude::*;
use mlua::prelude::*;

const CHUNK_SIZE: usize = 8;

#[derive(Debug, Clone)]
pub struct ChildProcessReader<R: AsyncRead>(pub R);

#[derive(Debug, Clone)]
pub struct ChildProcessWriter<W: AsyncWrite>(pub W);

impl<R: AsyncRead + Unpin> ChildProcessReader<R> {
    pub async fn read(&mut self, chunk_size: Option<usize>) -> LuaResult<Vec<u8>> {
        let mut buf = vec![0u8; chunk_size.unwrap_or(CHUNK_SIZE)];

        let read = self.0.read(&mut buf).await?;
        buf.truncate(read);

        Ok(buf)
    }

    pub async fn read_to_end(&mut self) -> LuaResult<Vec<u8>> {
        let mut buf = vec![];
        self.0.read_to_end(&mut buf).await?;

        Ok(buf)
    }
}

impl<R: AsyncRead + Unpin + 'static> LuaUserData for ChildProcessReader<R> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut(
            "read",
            |lua, mut this, chunk_size: Option<usize>| async move {
                let buf = this.read(chunk_size).await?;

                if buf.is_empty() {
                    return Ok(LuaValue::Nil);
                }

                Ok(LuaValue::String(lua.create_string(buf)?))
            },
        );

        methods.add_async_method_mut("readToEnd", |lua, mut this, ()| async move {
            Ok(lua.create_string(this.read_to_end().await?))
        });
    }
}

impl<W: AsyncWrite + Unpin> ChildProcessWriter<W> {
    pub async fn write(&mut self, data: BString) -> LuaResult<()> {
        self.0.write_all(data.as_ref()).await?;
        Ok(())
    }
}

impl<W: AsyncWrite + Unpin + 'static> LuaUserData for ChildProcessWriter<W> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut("write", |_, mut this, data| async move {
            this.write(data).await
        });
    }
}
