import os
import sys
import logging
import discord
from discord.ext import commands
import asyncio
from cogs.utils.db import DB

extensions = [
    'cogs.color_commands',
    'cogs.talking_commands'
]

@commands.command()
async def ping(ctx):
    logging.info("Got ping!")
    await ctx.reply('pong!')
    pass

@commands.command()
@commands.is_owner()
async def load_ext(ctx, name):
    logging.info("load extension: {}".format(name))
    ctx.bot.load_extension(name)

@commands.command()
@commands.is_owner()
async def reload_ext(ctx, name):
    logging.info("reload extension: {}".format(name))
    ctx.bot.reload_extension(name)

@commands.command()
@commands.is_owner()
async def unload_ext(ctx, name):
    logging.info("unload extension: {}".format(name))
    ctx.bot.unload_extension(name)

def run_bot(config):
    logging.basicConfig(level=logging.INFO)

    options = {}
    if 'https_proxy' in os.environ:
        logging.info('Use proxy {}'.format(os.environ['https_proxy']))
        options['proxy'] = os.environ['https_proxy']

    loop = asyncio.get_event_loop()
    loop.run_until_complete(DB.connect(config['database_url']))

    bot = commands.Bot('$', loop=loop, **options)
    bot.add_command(ping)
    bot.add_command(load_ext)
    bot.add_command(reload_ext)
    bot.add_command(unload_ext)
    for ext in extensions:
        bot.load_extension(ext)

    bot.run(config['token'])

def main():
    config = {
        'token': os.environ['DC_TOKEN'],
        'database_url': os.environ['DATABASE_URL']
    }
    run_bot(config)


if __name__ == '__main__':
    main()
