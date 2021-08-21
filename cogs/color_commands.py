import discord
from discord import Role
from discord.ext import commands, tasks
from discord.ext.commands import Bot, Context
import logging
from os import path
import pickle
from datetime import date, datetime, timedelta, time
import asyncio

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


class ColorRandomData:
    def __init__(self):
        self.color_data = {}
        if path.exists('color_data.pickle'):
            log.info('Loading roles')
            with open('color_data.pickle', 'rb') as f:
                self.color_data = pickle.load(f)
            log.info('Loaded {} roles'.format(len(self.color_data)))

    def save_color_data(self):
        log.info('Saving {} roles'.format(len(self.color_data)))
        with open('color_data.pickle', 'wb') as f:
            pickle.dump(self.color_data, f)

    def reg_role(self, guild: int, role: int):
        if guild not in self.color_data:
            self.color_data[guild] = {}
        if role not in self.color_data[guild]:
            self.color_data[guild][role] = {
                'offset': 0
            }
        self.save_color_data()

    def unreg_role(self, guild: int, role: int):
        ret = False
        if guild in self.color_data:
            if role in self.color_data[guild]:
                del self.color_data[guild][role]
                ret = True
                if len(self.color_data[guild]) == 0:
                    del self.color_data[guild]
        self.save_color_data()
        return ret

    def next_color(self, guild: int, role: int):
        ret = False
        if guild in self.color_data:
            if role in self.color_data[guild]:
                self.color_data[guild][role]['offset'] += 1
                ret = True
        self.save_color_data()
        return ret

    def today(self):
        return date.today().toordinal()

    def get_color(self, role: int, offset: int):
        return get_hashed_color(role, offset, self.today())

    def get_all_colors(self):
        today = self.today()
        ret = []
        for g, d1 in self.color_data.items():
            for r, d2 in d1.items():
                offset = d2['offset']
                ret.append((g, r, get_hashed_color(r, offset, today)))
        return ret

    def get_waiting_time(self):
        tomorrow = datetime.combine(date.today(), time()) + timedelta(days=1)
        now = datetime.now()
        return tomorrow - now + timedelta(seconds=30)

    def get_reg_list(self):
        today = self.today()
        ret = []
        for g, d1 in self.color_data.items():
            for r, d2 in d1.items():
                offset = d2['offset']
                ret.append((g, r, offset))
        return ret


class ColorRandomChange(commands.Cog):
    def __init__(self, bot: Bot):
        self.bot = bot
        self.color = ColorRandomData()
        self.update_routing.start()

    def cog_unload(self):
        self.update_routing.cancel()

    @commands.command()
    async def colorreg(self, ctx: Context, *, role: Role):
        async with ctx.typing():
            log.info('Reg this role: {}'.format(role))
            self.color.reg_role(ctx.guild.id, role.id)
            await self.update_all_colors()
            await ctx.reply("OK", delete_after=30)
            await ctx.message.delete(delay=30)
        pass

    @commands.command()
    async def colorunreg(self, ctx: Context, *, role: Role):
        async with ctx.typing():
            log.info('Unreg this role: {}'.format(role))
            ret = self.color.unreg_role(ctx.guild.id, role.id)
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
            ret = self.color.next_color(ctx.guild.id, role.id)
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
        for i, (gid, rid, offset) in enumerate(self.color.get_reg_list()):
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
        for gid, rid, color in self.color.get_all_colors():
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
