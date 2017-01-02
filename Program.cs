using System;
using System.Collections.Generic;

namespace ConsoleApplication
{
    class DwarfAction
    {

    }

    interface IPlayer
    {
        string Name { get; }
        DwarfAction PlaceDwarf(IEnumerable<DwarfAction> options);
    }

    class RandomPlayer: IPlayer
    {
        private readonly string _name;

        public RandomPlayer(string name)
        {
            _name = name;
        }

        public string Name { get { return _name;} }

        public DwarfAction PlaceDwarf(IEnumerable<DwarfAction> options)
        {
            return null;
        }

    }

    class Dwarf
    {

    }
    
    // TODO: Call Area?
    class PlayerState
    {
        private List<Dwarf> _dwarfs = new List<Dwarf>();
        private IPlayer _player;

        public PlayerState(IPlayer player)
        {
            _player = player;
        }

        public bool HasDwarf() { return false; }
        
        public IPlayer Player { get { return _player; } }
    }


    class Game
    {
        private int _turn = 0;
        public bool InProgress { get { return _turn < 1; } }
        public int Turn { get { return _turn; } }

        public void EndTurn() { _turn++; }

        public int Score(PlayerState area)
        {
            return 0;
        }
    }

    public class Program
    {
        public static void Main(string[] args)
        {
            PlayerState[] players = new []{ new PlayerState(new RandomPlayer("X")), new PlayerState(new RandomPlayer("Y")) };
            var game = new Game();

            while (game.InProgress)
            {
                Console.WriteLine("Starting turn " + game.Turn);
                // Action phase
                bool didPlace;
                do
                {
                    didPlace = false;
                    foreach (var p in players)
                    {
                        if (p.HasDwarf()) {
                            var options = new DwarfAction[0];
                            p.Player.PlaceDwarf(options);
                            didPlace = true;
                        }
                    }
                } while (didPlace);

                // Harvest phase

                // End turn
                game.EndTurn();
            }
            // Score
            foreach (var player in players)
            {
                Console.WriteLine("Player {0}: {1}", player.Player.Name, game.Score(player));
            }
        }
    }
}
