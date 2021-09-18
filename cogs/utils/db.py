import asyncpg

class DB():
    _uri: str = None
    _conn: asyncpg.Connection = None

    @classmethod
    def set_uri(cls, uri):
        cls._uri = uri

    @classmethod
    async def conn(cls):
        if (not cls._conn) or cls._conn.is_closed():
            await cls.connect()

        return cls._conn

    @classmethod
    async def connect(cls):
        conn = await asyncpg.connect(cls._uri)
        cls._conn = conn
        return conn










