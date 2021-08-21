import discord
from discord import TextChannel, Thread
from discord.ext import commands
from discord.ext.commands import Bot, Context
import logging

log = logging.getLogger('talk')


class TalkingCommands(commands.Cog):
    def __init__(self, bot: Bot):
        self.bot = bot

    @commands.Cog.listener()
    async def on_message(self, message):
        log.info('{}: {}'.format(message.author, message.content))
        pass

    @commands.command()
    @commands.is_owner()
    async def botsend(self, ctx: Context, channel, *, message):
        # log.info('channel: %d, msg: %s', channel.id, message)
        channel: TextChannel = ctx.bot.get_channel(int(channel))
        await channel.send(message)
        pass

    @commands.command()
    @commands.is_owner()
    async def thread_join(self, ctx: Context, channel: int, thread: int):
        channel: TextChannel = self.bot.get_channel(channel)
        thread: Thread = channel.get_thread(thread)
        await thread.join()

    @commands.command()
    @commands.is_owner()
    async def thread_send(self, ctx: Context, channel: int, thread: int, *, message):
        channel: TextChannel = self.bot.get_channel(channel)
        thread: Thread = channel.get_thread(thread)
        await thread.send(message)


def setup(bot):
    bot.add_cog(TalkingCommands(bot))
