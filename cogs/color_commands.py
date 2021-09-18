import discord
from discord import Role
from discord.ext import commands, tasks
from discord.ext.commands import Bot, Context
import logging
from os import path
import pickle
from datetime import date, datetime, timedelta, time, timezone
import asyncio
import asyncpg

from .utils.db import DB

log = logging.getLogger('color')


def get_random_color(id: int):
    id %= 1000000
    hue = id * 0.618033988749895
    hue %= 1.0
    sat = id * 0.377846739793041
    sat = (sat % 0.5) + 0.15
    return discord.Color.from_hsv(hue, sat, 0.95)


def get_hashed_color(role: int, offset: int, days: int):
    return get_random_color(hash((role, offset, days)))

class ColorRandomDataPG:
    def __init__(self):
        asyncio.get_event_loop().run_until_complete(self.init_db())
        self.tzinfo = timezone(timedelta(hours=8))


    async def init_db(self):
        conn = DB.conn()
        await conn.execute('''CREATE TABLE IF NOT EXISTS color_random_data(
                                guild bigint,
                                role bigint,
                                shift int
                           )''')
        log.info('Create table color_random_data')


    async def check_exists(self, guild: int, role: int):
        conn = DB.conn()
        count = await conn.fetchval(r'''
            SELECT COUNT(*)
            FROM color_random_data
            WHERE guild=$1 AND role=$2
        ''', guild, role)
        log.info("Check exist %d", count)
        return count > 0


    async def reg_role(self, guild: int, role: int):
        log.info('reg_role %d, %d', guild, role)
        ret = False

        conn = DB.conn()
        if not await self.check_exists(guild, role):
            await conn.execute('''
                INSERT INTO color_random_data
                VALUES ($1, $2, 0)
            ''', guild, role)
            ret = True
        return ret

    async def unreg_role(self, guild: int, role: int):
        log.info('unreg_role %d, %d', guild, role)
        ret = False

        conn = DB.conn()
        if await self.check_exists(guild, role):
            await conn.execute('''
                DELETE FROM color_random_data
                WHERE guild=$1 AND role=$2
            ''', guild, role)
            ret = True
        return ret

    async def next_color(self, guild: int, role: int):
        log.info('next_color %d, %d', guild, role)
        ret = False

        conn = DB.conn()
        if await self.check_exists(guild, role):
            await conn.execute('''
                UPDATE color_random_data
                SET shift = shift + 1
                WHERE guild=$1 AND role=$2
            ''', guild, role)
            ret = True
        return ret

    def today(self):
        return datetime.now(tz=self.tzinfo).date()

    def today_ord(self):
        return self.today().toordinal()

    def get_color(self, role: int, offset: int):
        return get_hashed_color(role, offset, self.today_ord())

    async def get_all_colors(self):
        today_ord= self.today_ord()
        conn = DB.conn()
        records = await conn.fetch('''
            SELECT guild, role, shift
            FROM color_random_data
        ''')
        ret = [(g, r, get_hashed_color(r, s, today_ord)) for g, r, s in records]
        log.info('get_color Get {} roles'.format(len(ret)))

        return ret

    def get_waiting_time(self):
        tomorrow = datetime.combine(self.today(), time(), tzinfo=self.tzinfo) + timedelta(days=1)
        now = datetime.now(tz=self.tzinfo)
        return tomorrow - now + timedelta(seconds=30)

    async def get_reg_list(self):
        conn = DB.conn()
        records = await conn.fetch('''
            SELECT guild, role, shift
            FROM color_random_data
        ''')

        ret = [(g, r, s) for g, r, s in records]
        log.info('get_list Get {} roles'.format(len(ret)))
        return ret

class ColorRandomChange(commands.Cog):
    def __init__(self, bot: Bot):
        self.bot = bot
        self.color = ColorRandomDataPG()
        self.update_routing.start()

    def cog_unload(self):
        self.update_routing.cancel()

    @commands.command()
    async def colorreg(self, ctx: Context, *, role: Role):
        async with ctx.typing():
            log.info('Reg this role: {}'.format(role))
            await self.color.reg_role(ctx.guild.id, role.id)
            await self.update_all_colors()
            await ctx.reply("OK", delete_after=30)
            await ctx.message.delete(delay=30)
        pass

    @commands.command()
    async def colorunreg(self, ctx: Context, *, role: Role):
        async with ctx.typing():
            log.info('Unreg this role: {}'.format(role))
            ret = await self.color.unreg_role(ctx.guild.id, role.id)
            if ret:
                message = 'OK'
            else:
                message = 'This role is not found.'
            await self.update_all_colors()
            await ctx.reply(message, delete_after=30)
            await ctx.message.delete(delay=30)

    @commands.command()
    async def nextcolor(self, ctx: Context, *, role: Role):
        async with ctx.typing():
            log.info("Next color on role: {}".format(role))
            ret = await self.color.next_color(ctx.guild.id, role.id)
            if ret:
                message = 'OK'
            else:
                message = 'This role is not found.'
            await self.update_all_colors()
            await ctx.reply(message, delete_after=30)
            await ctx.message.delete(delay=30)

    @tasks.loop(minutes=1)
    async def update_routing(self):
        while True:
            log.info('Routing update color')
            await self.update_all_colors()
            delta = self.color.get_waiting_time()
            log.info('Next routing: {} secs'.format(delta.total_seconds()))
            await asyncio.sleep(delta.total_seconds())
        pass

    @commands.command()
    @commands.is_owner()
    async def listregs(self, ctx: Context):
        message = ''
        for i, (gid, rid, offset) in enumerate(await self.color.get_reg_list()):
            guild = None
            role = None
            guild = self.bot.get_guild(gid)
            if guild != None:
                role = guild.get_role(rid)
            message += '%d. [%s] [%s] offset:%d\n' % (i, guild, role, offset)

        await ctx.reply(message)
        pass

    async def update_all_colors(self):
        log.info('Update all colors')
        for gid, rid, color in await self.color.get_all_colors():
            guild = self.bot.get_guild(gid)
            if guild == None:
                continue
            role = guild.get_role(rid)
            if role == None:
                continue
            if color != role.color:
                await role.edit(color=color, reason='Random coloring.')


@commands.command()
async def color(ctx, name):
    logging.info("Color reg")
    pass


def setup(bot):
    bot.add_command(color)
    bot.add_cog(ColorRandomChange(bot))
