namespace App;

public class Account
{
    public const double Rate = 0.05;
    private double _balance;

    public Account(string id) { }

    public double Balance()
    {
        return _balance;
    }
}

public interface IGreeter
{
    string Greet();
}

public enum Status { Open, Closed }
