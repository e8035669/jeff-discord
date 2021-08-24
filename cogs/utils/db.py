import asyncpg

class DB():
    @classmethod
    async def connect(cls, uri):
        conn = await asyncpg.connect(uri)
        cls._conn = conn
        return conn






